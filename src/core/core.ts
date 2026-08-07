interface D2FDocumentContext {
    readonly id?: string;
}

interface D2FStorageHandler {
    readonly registerSaveHandler: (handler: () => Record<string, unknown> | undefined) => void;
    readonly registerLoadHandler: (handler: (state: Record<string, unknown>) => void) => void;
    readonly loadState: () => void;
    readonly saveState: () => void;
}

interface D2FCoreHelpers {
    readonly isRecord: (val: unknown) => val is Record<string, unknown>;
}

interface Window {
    d2f?: {
        document?: D2FDocumentContext;
        storage?: D2FStorageHandler;
        utils?: D2FCoreHelpers;
    };
}

(() => {
    const saveHandlers = new Set<() => Record<string, unknown> | undefined>();
    const loadHandlers = new Set<(state: Record<string, unknown>) => void>();

    function isRecord(val: unknown): val is Record<string, unknown> {
        return typeof val === 'object' && val !== null && !Array.isArray(val);
    }

    function getStateKey(): string {
        const docId = window.d2f?.document?.id ?? '';
        return docId !== '' ? `d2f_state_${docId}` : 'd2f_state_global';
    }

    function loadState(): void {
        const key = getStateKey();
        try {
            const raw = localStorage.getItem(key);
            if (!raw) return;
            const parsed: unknown = JSON.parse(raw);
            if (!isRecord(parsed)) return;

            loadHandlers.forEach((handler) => {
                try {
                    handler(parsed);
                } catch (e) {
                    console.warn('Failed to execute load handler', e);
                }
            });
        } catch (e) {
            console.warn(`Failed to load state from localStorage [key: ${key}]`, e);
        }
    }

    function saveState(): void {
        const combinedState: Record<string, unknown> = {};
        saveHandlers.forEach((handler) => {
            try {
                const partialState = handler();
                if (partialState && typeof partialState === 'object') {
                    Object.assign(combinedState, partialState);
                }
            } catch (e) {
                console.warn('Failed to collect state from handler', e);
            }
        });

        const key = getStateKey();
        try {
            localStorage.setItem(key, JSON.stringify(combinedState));
        } catch (e) {
            console.warn(`Failed to save state to localStorage [key: ${key}]`, e);
        }
    }

    const storage = {
        registerSaveHandler(handler: () => Record<string, unknown> | undefined): void {
            saveHandlers.add(handler);
        },
        registerLoadHandler(handler: (state: Record<string, unknown>) => void): void {
            loadHandlers.add(handler);
        },
        loadState,
        saveState,
    } satisfies D2FStorageHandler;

    const utils = {
        isRecord,
    } satisfies D2FCoreHelpers;

    window.d2f = window.d2f ?? {};
    window.d2f.storage = storage;
    window.d2f.utils = utils;
})();
