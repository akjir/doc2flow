import type { Core } from './core/core.js';
import type { Storage } from './core/storage.js';
import type { Collapse } from './core/collapse.js';
import type { Comments } from './core/comments.js';
import type { Export } from './core/export.js';
import type { Fields } from './core/fields.js';
import type { Search } from './core/search.js';
import type { Utils } from './core/utils.js';
import type { Tasks } from './features/tasks.js';
import type { Code } from './features/code.js';
import type { Images } from './features/images.js';
import type { Toc } from './features/toc.js';

export interface D2FI18nDict {
    readonly sections_visible?: string;
    readonly comment_placeholder?: string;
    readonly progress_template?: string;
    readonly setup_in_progress?: string;
    readonly setup_completed?: string;
    readonly date_placeholder?: string;
    readonly [key: string]: string | undefined;
}

export interface D2FNamespace {
    core: Core;
    storage: Storage;
    collapse: Collapse;
    comments: Comments;
    export: Export;
    fields: Fields;
    search: Search;
    utils: Utils;
    tasks?: Tasks;
    code?: Code;
    images?: Images;
    toc?: Toc;
}

declare global {
    interface Window {
        D2F_DOC_ID?: string;
        D2F_I18N?: D2FI18nDict;
        d2f: D2FNamespace;
        exportPDF?: () => void;
        saveDocumentState?: () => void;
        resetAll?: () => void;
        copyCode?: (btn: HTMLElement | null) => Promise<void> | void;
        openLightbox?: (imgSrc: string) => void;
        closeLightbox?: () => void;
    }
}

