// Types

type State = Record<string, unknown>;
type SaveHandler = () => State;
type LoadHandler = (state: State) => boolean;

// Constants

const saveHandlers = new Set<SaveHandler>();
const loadHandlers = new Set<LoadHandler>();

// Handler Functions

export function registerSaveHandler(handler: SaveHandler): void {
    saveHandlers.add(handler);
}

export function registerLoadHandler(handler: LoadHandler): void {
    loadHandlers.add(handler);
}

// Exported Functions

export function loadState(
    styleItemFn?: (cb: HTMLInputElement) => void,
    getOrCreateCommentBoxFn?: (item: HTMLElement, initialValue?: string) => void
): void {
    try {
        const raw = localStorage.getItem(getStateKey());
        if (!raw) return;

        const data = parseRawStateJson(raw);
        if (!data) return;

        for (const handler of loadHandlers) {
            try {
                if (handler(data)) {
                    return;
                }
            } catch (e) {
                console.warn('Failed to execute load handler', e);
            }
        }

        loadStateOld(data, styleItemFn, getOrCreateCommentBoxFn);
    } catch (e) {
        console.warn('Failed to load state from localStorage', e);
    }
}

export function saveState(): void {
    let combinedState: State = {};

    try {
        const oldState = saveStateOld();
        combinedState = { ...combinedState, ...oldState };
    } catch (e) {
        console.warn('Failed to collect state from saveStateOld', e);
    }

    for (const handler of saveHandlers) {
        try {
            const providerState = handler();
            combinedState = { ...combinedState, ...providerState };
        } catch (e) {
            console.warn('Failed to collect state from handler', e);
        }
    }

    try {
        localStorage.setItem(getStateKey(), JSON.stringify(combinedState));
    } catch (e) {
        console.warn('Failed to save state to localStorage', e);
    }
}

// Internal Functions




// -- old code 

export interface D2FState {
    readonly checks: Record<string, boolean>;
    readonly texts: Record<string, boolean>;
    readonly fields: Record<string, string>;
    readonly comments: Record<string, string>;
    readonly sections: Record<string, boolean>;
}


export function debounce<T extends (...args: readonly unknown[]) => void>(
    func: T,
    wait: number
): (...args: Parameters<T>) => void {
    let timeout: ReturnType<typeof setTimeout> | undefined;
    return function (...args: Parameters<T>): void {
        if (timeout !== undefined) {
            clearTimeout(timeout);
        }
        timeout = setTimeout(() => func(...args), wait);
    };
}

export function getStateKey(): string {
    const docId = window.D2F_DOC_ID ?? '';
    const rawFilename = window.location.pathname.split('/').pop() ?? 'index.html';
    const filename = decodeURIComponent(rawFilename);
    return 'd2f_state_' + (docId ? (docId + '_') : '') + filename;
}

function saveStateOld(): State {
    const state: Record<string, boolean> = {};
    document.querySelectorAll<HTMLInputElement>('.check-item input[type="checkbox"]').forEach((cb, index) => {
        const key = cb.id || ('cb_' + String(index));
        state[key] = cb.checked;
    });

    const textStates: Record<string, boolean> = {};
    document.querySelectorAll<HTMLElement>('.check-item.text-item, .check-item.simple-item').forEach((item, index) => {
        const key = item.id || ('txt_' + String(index));
        textStates[key] = item.classList.contains('checked');
    });

    const fields: Record<string, string> = {};
    document.querySelectorAll<HTMLInputElement>('input.persistent-field').forEach((input, index) => {
        const key = input.id || ('f_' + String(index));
        fields[key] = input.value;
    });

    const comments: Record<string, string> = {};
    document.querySelectorAll<HTMLElement>('.check-item').forEach((item, index) => {
        const input = item.querySelector<HTMLTextAreaElement>('.item-comment-input');
        if (input && input.value.trim() !== '') {
            const key = item.id || ('item_' + String(index));
            comments[key] = input.value;
        }
    });

    const sections: Record<string, boolean> = {};
    document.querySelectorAll<HTMLElement>('.d2f-section, .section').forEach((sec, index) => {
        const body = sec.querySelector<HTMLElement>('.sb');
        if (body) {
            const key = sec.id || ('sec_' + String(index));
            sections[key] = body.classList.contains('collapsed');
        }
    });

    return {
        checks: state,
        texts: textStates,
        fields: fields,
        comments: comments,
        sections: sections
    };
}

export const saveStateDebounced = debounce(saveState, 300);

export function syncFieldPair(id1: string, id2: string, sourceInput?: HTMLInputElement): void {
    const raw1 = document.getElementById(id1);
    const raw2 = document.getElementById(id2);
    const el1 = raw1 instanceof HTMLInputElement ? raw1 : null;
    const el2 = raw2 instanceof HTMLInputElement ? raw2 : null;
    if (!el1 || !el2) return;

    if (sourceInput === el1) {
        el2.value = el1.value;
    } else if (sourceInput === el2) {
        el1.value = el2.value;
    } else {
        if (el1.value && !el2.value) el2.value = el1.value;
        else if (el2.value && !el1.value) el1.value = el2.value;
        else if (el1.value) el2.value = el1.value;
    }
}

export function syncLinkedFields(sourceInput?: HTMLInputElement): void {
    syncFieldPair('f_info_agent', 'f_sign_agent', sourceInput);
    syncFieldPair('f_info_date', 'f_sign_date', sourceInput);
}

export function formatDateFromTemplate(now: Date, template?: string): string | null {
    if (!template || typeof template !== 'string') return null;

    const tokenMap: Record<string, string> = {
        'YYYY': String(now.getFullYear()),
        'YY': String(now.getFullYear()).slice(-2),
        'MM': String(now.getMonth() + 1).padStart(2, '0'),
        'DD': String(now.getDate()).padStart(2, '0'),
        'M': String(now.getMonth() + 1),
        'D': String(now.getDate())
    };

    const regex = /YYYY|YY|MM|DD|M|D/gi;
    let hasMatches = false;

    const formatted = template.replace(regex, (match) => {
        hasMatches = true;
        return tokenMap[match.toUpperCase()] ?? match;
    });

    return (hasMatches && !/[A-Za-z]/.test(formatted)) ? formatted : null;
}

export function getTodayFormatted(): string {
    const i18n = window.D2F_I18N ?? {};
    const now = new Date();
    try {
        const fromTemplate = formatDateFromTemplate(now, i18n.date_placeholder);
        if (fromTemplate) return fromTemplate;
    } catch (e) {
        console.warn('Failed to format date', e);
    }
    return now.toLocaleDateString(navigator.language || undefined);
}

export function checkDateShortcut(input: HTMLInputElement): boolean {
    if (typeof input.value !== 'string') return false;
    if (input.value.trim().toLowerCase() === 'today') {
        input.value = getTodayFormatted();
        return true;
    }
    return false;
}

function parseRawStateJson(json: string): State | null {
    try {
        const parsed: unknown = JSON.parse(json);
        if (typeof parsed === 'object' && parsed !== null && !Array.isArray(parsed)) {
            return parsed as State;
        }
    } catch {
        // ignore JSON parse error
    }
    return null;
}

function loadStateOld(
    data: State,
    styleItemFn?: (cb: HTMLInputElement) => void,
    getOrCreateCommentBoxFn?: (item: HTMLElement, initialValue?: string) => void
): void {
    const legacyData = data as Partial<D2FState>;

    if (legacyData.checks) {
        document.querySelectorAll<HTMLInputElement>('.check-item input[type="checkbox"]').forEach((cb, index) => {
            const key = cb.id || ('cb_' + String(index));
            const val = legacyData.checks?.[key];
            if (val !== undefined) {
                cb.checked = val;
                if (styleItemFn) styleItemFn(cb);
            }
        });
    }
    if (legacyData.texts) {
        document.querySelectorAll<HTMLElement>('.check-item.text-item, .check-item.simple-item').forEach((item, index) => {
            const key = item.id || ('txt_' + String(index));
            const val = legacyData.texts?.[key];
            if (val !== undefined) {
                item.classList.toggle('checked', val);
            }
        });
    }
    if (legacyData.fields) {
        document.querySelectorAll<HTMLInputElement>('input.persistent-field').forEach((input, index) => {
            const key = input.id || ('f_' + String(index));
            const val = legacyData.fields?.[key];
            if (val !== undefined) {
                input.value = val;
            }
        });
    }
    if (legacyData.comments && getOrCreateCommentBoxFn) {
        document.querySelectorAll<HTMLElement>('.check-item').forEach((item, index) => {
            const key = item.id || ('item_' + String(index));
            const val = legacyData.comments?.[key];
            if (val !== undefined) {
                getOrCreateCommentBoxFn(item, val);
            }
        });
    }
    if (legacyData.sections) {
        document.querySelectorAll<HTMLElement>('.d2f-section, .section').forEach((sec, index) => {
            const key = sec.id || ('sec_' + String(index));
            const shouldCollapse = legacyData.sections?.[key];
            if (shouldCollapse === undefined) return;
            const body = sec.querySelector<HTMLElement>('.sb');
            const sh = sec.querySelector<HTMLElement>('.sh');
            if (!body) return;
            body.classList.toggle('collapsed', shouldCollapse);
            if (sh) {
                sh.setAttribute('aria-expanded', shouldCollapse ? 'false' : 'true');
                const toggler = sh.querySelector<HTMLElement>('.stog');
                if (toggler) toggler.innerHTML = shouldCollapse ? '&#9650;' : '&#9660;';
            }
        });
    }
    syncLinkedFields();
}
