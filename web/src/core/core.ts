import { saveState } from './storage.js';

// Types

export type ResetHandler = () => void;

// Constants

const resetHandlers = new Set<ResetHandler>();

// Handler Functions

export function registerResetHandler(handler: ResetHandler): void {
    resetHandlers.add(handler);
}

// Exported Functions

export function resetAll(): void {
    const i18n = window.D2F_I18N;
    const confirmMsg = i18n?.confirm_reset;
    if (!confirmMsg) {
        console.error('Missing i18n translation key: confirm_reset');
        return;
    }
    if (!confirm(confirmMsg)) return;

    for (const handler of resetHandlers) {
        try {
            handler();
        } catch (e) {
            console.warn('Failed to execute reset handler', e);
        }
    }

    saveState();
}
