export interface Storage {
    registerSaveHandler(handler: SaveHandler): void;
    registerLoadHandler(handler: LoadHandler): void;
    loadState(): void;
    saveState(): void;
}

export type State = Record<string, unknown>;
export type SaveHandler = () => State;
export type LoadHandler = (state: State) => void;

const saveHandlers = new Set<SaveHandler>();
const loadHandlers = new Set<LoadHandler>();

window.d2f.storage = {
    registerSaveHandler,
    registerLoadHandler,
    loadState,
    saveState,
}

function registerSaveHandler(handler: SaveHandler): void {
    saveHandlers.add(handler);
}

function registerLoadHandler(handler: LoadHandler): void {
    loadHandlers.add(handler);
}

function loadState(): void {
    const key = getStateKey();
    try {
        const raw = localStorage.getItem(key);
        if (!raw) return;

        const parsed: unknown = JSON.parse(raw);
        if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) return;
        const state = parsed as State;

        for (const handler of loadHandlers) {
            try {
                handler(state);
            } catch (e) {
                console.warn('Failed to execute load handler', e);
            }
        }
    } catch (e) {
        console.warn(`Failed to load state from localStorage [key: ${key}]`, e);
    }
}

function saveState(): void {
    const combinedState: State = {};

    for (const handler of saveHandlers) {
        try {
            const providerState = handler();
            if (providerState && typeof providerState === 'object') {
                Object.assign(combinedState, providerState);
            }
        } catch (e) {
            console.warn('Failed to collect state from handler', e);
        }
    }

    const key = getStateKey();
    try {
        localStorage.setItem(key, JSON.stringify(combinedState));
    } catch (e) {
        console.warn(`Failed to save state to localStorage [key: ${key}]`, e);
    }
}


function getStateKey(): string {
    const docId = window.D2F_DOC_ID ?? '';
    const rawFilename = window.location.pathname.split('/').pop() ?? 'index.html';
    const filename = decodeURIComponent(rawFilename);
    return 'd2f_state_' + (docId ? `${docId}_` : '') + filename;
}