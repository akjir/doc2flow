export type ResetHandler = () => void;

export interface Core {
    registerResetHandler(handler: ResetHandler): void;
    resetAll(): void;
}

const resetHandlers = new Set<ResetHandler>();

window.d2f.core = {
    registerResetHandler,
    resetAll,
};

function registerResetHandler(handler: ResetHandler): void {
    resetHandlers.add(handler);
}

function resetAll(): void {
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

    window.d2f.storage.saveState();
}
