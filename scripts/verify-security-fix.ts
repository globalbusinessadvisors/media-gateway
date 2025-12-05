#!/usr/bin/env node
/**
 * Security Fix Verification Script
 * Demonstrates that command injection vulnerability has been fixed
 */

import { runCommand } from '../src/utils/installer.js';

async function verifySecurityFix() {
  console.log('🔒 Security Fix Verification\n');

  // Test 1: Safe commands should work
  console.log('✅ Test 1: Safe commands');
  const safeCommands = [
    'npm --version',
    'node --version',
    'npx --version'
  ];

  for (const cmd of safeCommands) {
    try {
      await runCommand(cmd);
    console.log(`   ✓ ${cmd} - PASSED`);
  } catch (error) {
    // Command may fail if tool not installed, but validation should pass
    if (error.message.includes('not allowed') || error.message.includes('unsafe')) {
      console.log(`   ✗ ${cmd} - FAILED (validation error)`);
    } else {
      console.log(`   ✓ ${cmd} - PASSED (validation OK, tool may not be installed)`);
    }
    }
  }

  // Test 2: Malicious commands should be blocked
  console.log('\n🛡️  Test 2: Command injection protection');
  const maliciousCommands = [
    { cmd: 'npm install; rm -rf /', threat: 'Command chaining' },
    { cmd: 'npm install && whoami', threat: 'Command chaining (AND)' },
    { cmd: 'npm install | grep secret', threat: 'Pipe operator' },
    { cmd: 'npm install `whoami`', threat: 'Backtick injection' },
    { cmd: 'npm install $(whoami)', threat: 'Command substitution' },
    { cmd: 'npm install ${USER}', threat: 'Variable substitution' },
    { cmd: 'npm install > /etc/passwd', threat: 'Output redirection' },
    { cmd: 'bash -c "malicious"', threat: 'Disallowed command' },
  ];

  let blockedCount = 0;
  for (const { cmd, threat } of maliciousCommands) {
    try {
      await runCommand(cmd);
    console.log(`   ✗ ${threat} - FAILED (command was NOT blocked)`);
  } catch (error) {
    if (error.message.includes('not allowed') || error.message.includes('unsafe')) {
      console.log(`   ✓ ${threat} - BLOCKED`);
      blockedCount++;
    } else {
      console.log(`   ? ${threat} - UNKNOWN (${error.message})`);
    }
    }
  }

  // Test 3: Process cleanup handlers
  console.log('\n🧹 Test 3: Process cleanup handlers');
  const signalHandlers = [
    { signal: 'SIGINT' as const, name: 'Interrupt (Ctrl+C)' },
    { signal: 'SIGTERM' as const, name: 'Termination' },
    { signal: 'exit' as const, name: 'Exit' }
  ];

  for (const { signal, name } of signalHandlers) {
    const listeners = process.listeners(signal);
    if (listeners.length > 0) {
      console.log(`   ✓ ${name} handler registered`);
    } else {
      console.log(`   ✗ ${name} handler MISSING`);
    }
  }

  // Summary
  console.log('\n📊 Summary');
  console.log(`   Safe commands tested: ${safeCommands.length}`);
  console.log(`   Malicious commands blocked: ${blockedCount}/${maliciousCommands.length}`);
  console.log(`   Signal handlers registered: ${signalHandlers.filter(h => process.listeners(h.signal).length > 0).length}/${signalHandlers.length}`);

  if (blockedCount === maliciousCommands.length) {
    console.log('\n✅ SECURITY FIX VERIFIED - All command injection attempts blocked!');
    process.exit(0);
  } else {
    console.log('\n❌ SECURITY FIX INCOMPLETE - Some malicious commands were not blocked!');
    process.exit(1);
  }
}

// Run verification
verifySecurityFix().catch(error => {
  console.error('❌ Verification script failed:', error);
  process.exit(1);
});
