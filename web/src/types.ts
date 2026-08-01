export interface D2FI18nDict {
    readonly sections_visible?: string;
    readonly comment_placeholder?: string;
    readonly progress_template?: string;
    readonly setup_in_progress?: string;
    readonly setup_completed?: string;
    readonly date_placeholder?: string;
    readonly [key: string]: string | undefined;
}

export interface D2FState {
    readonly checks: Record<string, boolean>;
    readonly texts: Record<string, boolean>;
    readonly fields: Record<string, string>;
    readonly comments: Record<string, string>;
    readonly sections: Record<string, boolean>;
}

declare global {
    interface Window {
        D2F_DOC_ID?: string;
        D2F_I18N?: D2FI18nDict;
        exportPDF?: () => void;
        saveDocumentState?: () => void;
        resetAll?: () => void;
        copyCode?: (btn: HTMLElement | null) => void;
    }
}
