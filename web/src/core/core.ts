import './storage.js';
import './utils.js';
import { ExportType } from './export.js';
import './sections.js';
import './comments.js';
import './fields.js';
import './search.js';

export type ResetHandler = () => void;

export interface Core {
    registerResetHandler(handler: ResetHandler): void;
    resetAll(): void;
}

const resetHandlers = new Set<ResetHandler>();

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

window.d2f.core = {
    registerResetHandler,
    resetAll,
};

if (typeof window !== 'undefined') {
    window.exportPDF = () => window.d2f.export.export(ExportType.PDF);
    window.saveDocumentState = () => window.d2f.export.export(ExportType.DOCUMENT);
    window.resetAll = () => window.d2f.core.resetAll();
}
