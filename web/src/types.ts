import type { Core } from './core/core.ts';
import type { Storage } from './core/storage.ts';
import type { Export } from './core/export.ts';
import type { Utils } from './core/utils.ts';

declare global {
    interface Window {
        d2f: {
            core: Core;
            storage: Storage;
            export: Export,
            utils: Utils,
        }
    }
}

export interface D2FI18nDict {
    readonly sections_visible?: string;
    readonly comment_placeholder?: string;
    readonly progress_template?: string;
    readonly setup_in_progress?: string;
    readonly setup_completed?: string;
    readonly date_placeholder?: string;
    readonly [key: string]: string | undefined;
}

declare global {
    interface Window {
        D2F_DOC_ID?: string;
        D2F_I18N?: D2FI18nDict;
        exportPDF?: () => void;
        saveDocumentState?: () => void;
        resetAll?: () => void;
        copyCode?: (btn: HTMLElement | null) => Promise<void> | void;
        openLightbox?: (imgSrc: string) => void;
        closeLightbox?: () => void;
    }
}

