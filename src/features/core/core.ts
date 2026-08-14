interface D2fCoreState {
  readonly initialized: boolean;
  readonly name: string;
}

declare global {
  interface Window {
    d2f: Record<string, unknown>;
  }
}

(() => {
  const state: D2fCoreState = {
    initialized: true,
    name: 'core',
  };

  window.d2f = window.d2f || {};
  window.d2f['core'] = state;
})();
