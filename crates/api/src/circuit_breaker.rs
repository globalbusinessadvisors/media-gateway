use crate::config::{CircuitBreakerServiceConfig, Config};
use crate::error::{ApiError, ApiResult};
use failsafe::{CircuitBreaker, Config as FailsafeConfig, Error as FailsafeError};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, warn};

pub struct CircuitBreakerManager {
    breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    config: Arc<Config>,
}

impl CircuitBreakerManager {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn get_or_create(&self, service: &str) -> Arc<CircuitBreaker> {
        let breakers = self.breakers.read().await;

        if let Some(breaker) = breakers.get(service) {
            return Arc::new(breaker.clone());
        }

        drop(breakers);

        let mut breakers = self.breakers.write().await;

        // Double-check after acquiring write lock
        if let Some(breaker) = breakers.get(service) {
            return Arc::new(breaker.clone());
        }

        let breaker = self.create_circuit_breaker(service);
        breakers.insert(service.to_string(), breaker.clone());

        Arc::new(breaker)
    }

    fn create_circuit_breaker(&self, service: &str) -> CircuitBreaker {
        let service_config = self
            .config
            .circuit_breaker
            .services
            .get(service)
            .cloned()
            .unwrap_or_else(|| CircuitBreakerServiceConfig {
                failure_threshold: 10,
                timeout_seconds: 2,
                error_rate_threshold: 0.5,
            });

        let failsafe_config = FailsafeConfig::new()
            .failure_threshold(service_config.failure_threshold)
            .timeout(Duration::from_secs(service_config.timeout_seconds))
            .error_rate_threshold(service_config.error_rate_threshold);

        debug!(
            service = service,
            failure_threshold = service_config.failure_threshold,
            timeout_seconds = service_config.timeout_seconds,
            error_rate = service_config.error_rate_threshold,
            "Created circuit breaker"
        );

        CircuitBreaker::new(failsafe_config)
    }

    pub async fn call<F, T, E>(
        &self,
        service: &str,
        operation: F,
    ) -> ApiResult<T>
    where
        F: FnOnce() -> Result<T, E> + Send,
        E: std::error::Error + Send + Sync + 'static,
    {
        if !self.config.circuit_breaker.enabled {
            return operation().map_err(|e| {
                ApiError::ProxyError(format!("Service {} error: {}", service, e))
            });
        }

        let breaker = self.get_or_create(service).await;

        match breaker.call(operation) {
            Ok(result) => Ok(result),
            Err(FailsafeError::Rejected) => {
                warn!(service = service, "Circuit breaker open");
                Err(ApiError::CircuitBreakerOpen(service.to_string()))
            }
            Err(FailsafeError::Inner(e)) => {
                warn!(service = service, error = %e, "Service call failed");
                Err(ApiError::ProxyError(format!("Service {} error: {}", service, e)))
            }
        }
    }

    pub async fn get_state(&self, service: &str) -> Option<String> {
        let breakers = self.breakers.read().await;
        breakers.get(service).map(|b| {
            if b.is_open() {
                "open"
            } else if b.is_half_open() {
                "half_open"
            } else {
                "closed"
            }
            .to_string()
        })
    }

    pub async fn get_all_states(&self) -> HashMap<String, String> {
        let breakers = self.breakers.read().await;
        breakers
            .iter()
            .map(|(service, breaker)| {
                let state = if breaker.is_open() {
                    "open"
                } else if breaker.is_half_open() {
                    "half_open"
                } else {
                    "closed"
                }
                .to_string();
                (service.clone(), state)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_disabled() {
        let mut config = Config::default();
        config.circuit_breaker.enabled = false;
        let manager = CircuitBreakerManager::new(Arc::new(config));

        let result = manager
            .call("test", || Ok::<_, std::io::Error>("success"))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = Config::default();
        let manager = CircuitBreakerManager::new(Arc::new(config));

        let result = manager
            .call("discovery", || Ok::<_, std::io::Error>("success"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
}
