export type Language = {
    dictionary: Dictionary;
};

export interface Dictionary {
    readonly sections_visible?: string;
    readonly comment_placeholder?: string;
    readonly progress_template?: string;
    readonly setup_in_progress?: string;
    readonly setup_completed?: string;
    readonly date_placeholder?: string;
    readonly [key: string]: string | undefined;
}
