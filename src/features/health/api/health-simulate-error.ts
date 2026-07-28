let simulateHealthError = false;

export function setHealthSimulateError(enabled: boolean) {
  simulateHealthError = enabled;
}

export function isHealthSimulateErrorEnabled() {
  return simulateHealthError;
}
