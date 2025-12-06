use crate::config::{CircuitBreakerServiceConfig, Config};
use crate::error::{ApiError, ApiResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

#[derive(Clone, Debug)]
enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

#[derive(Clone)]
struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    success_count: Arc<RwLock<u32>>,
    config: CircuitBreakerServiceConfig,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerServiceConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            config,
        }
    }

    async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, CircuitState::Open { .. })
    }

    #[allow(dead_code)]
    async fn is_half_open(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, CircuitState::HalfOpen)
    }

    async fn check_and_update_state(&self) {
        let mut state = self.state.write().await;

        match &*state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= Duration::from_secs(self.config.timeout_seconds) {
                    *state = CircuitState::HalfOpen;
                    debug!("Circuit breaker entering half-open state");
                }
            }
            _ => {}
        }
    }

    async fn record_success(&self) {
        let mut success_count = self.success_count.write().await;
        let mut failure_count = self.failure_count.write().await;
        let mut state = self.state.write().await;

        *success_count += 1;

        match &*state {
            CircuitState::HalfOpen => {
                // If we get a success in half-open, close the circuit
                *state = CircuitState::Closed;
                *failure_count = 0;
                *success_count = 0;
                debug!("Circuit breaker closed after successful half-open request");
            }
            CircuitState::Closed => {
                // Reset failure count on success
                *failure_count = 0;
            }
            _ => {}
        }
    }

    async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        let mut state = self.state.write().await;

        *failure_count += 1;

        match &*state {
            CircuitState::Closed => {
                if *failure_count >= self.config.failure_threshold {
                    *state = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                    warn!("Circuit breaker opened due to failures");
                }
            }
            CircuitState::HalfOpen => {
                // If we fail in half-open, go back to open
                *state = CircuitState::Open {
                    opened_at: Instant::now(),
                };
                warn!("Circuit breaker re-opened after failed half-open request");
            }
            _ => {}
        }
    }

    async fn get_state_string(&self) -> String {
        let state = self.state.read().await;
        match &*state {
            CircuitState::Closed => "closed".to_string(),
            CircuitState::Open { .. } => "open".to_string(),
            CircuitState::HalfOpen => "half_open".to_string(),
        }
    }
}

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

    pub async fn get_or_create(&self, service: &str) -> CircuitBreaker {
        let breakers = self.breakers.read().await;

        if let Some(breaker) = breakers.get(service) {
            return breaker.clone();
        }

        drop(breakers);

        let mut breakers = self.breakers.write().await;

        // Double-check after acquiring write lock
        if let Some(breaker) = breakers.get(service) {
            return breaker.clone();
        }

        let breaker = self.create_circuit_breaker(service);
        breakers.insert(service.to_string(), breaker.clone());

        breaker
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

        debug!(
            service = service,
            failure_threshold = service_config.failure_threshold,
            timeout_seconds = service_config.timeout_seconds,
            error_rate = service_config.error_rate_threshold,
            "Created circuit breaker"
        );

        CircuitBreaker::new(service_config)
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

        // Check and potentially update the state
        breaker.check_and_update_state().await;

        // If circuit is open, reject the request
        if breaker.is_open().await {
            warn!(service = service, "Circuit breaker open");
            return Err(ApiError::CircuitBreakerOpen(service.to_string()));
        }

        // Execute the operation
        match operation() {
            Ok(result) => {
                breaker.record_success().await;
                Ok(result)
            }
            Err(e) => {
                breaker.record_failure().await;
                warn!(service = service, error = %e, "Service call failed");
                Err(ApiError::ProxyError(format!("Service {} error: {}", service, e)))
            }
        }
    }

    pub async fn get_state(&self, service: &str) -> Option<String> {
        let breakers = self.breakers.read().await;
        if let Some(breaker) = breakers.get(service) {
            Some(breaker.get_state_string().await)
        } else {
            None
        }
    }

    pub async fn get_all_states(&self) -> HashMap<String, String> {
        let breakers = self.breakers.read().await;
        let mut states = HashMap::new();

        for (service, breaker) in breakers.iter() {
            states.insert(service.clone(), breaker.get_state_string().await);
        }

        states
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
