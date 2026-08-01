import { registerSaveHandler, registerLoadHandler } from './storage.js';

function isRecord(val: unknown): val is Record<string, unknown> {
    return typeof val === 'object' && val !== null && !Array.isArray(val);
}

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
        const key = match.toUpperCase();
        const value = tokenMap[key];
        return value ?? match;
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

export function saveFields(): Record<string, unknown> {
    const fields: Record<string, string> = {};
    document.querySelectorAll<HTMLInputElement>('input.persistent-field').forEach((input, index) => {
        const key = input.id || ('f_' + String(index));
        fields[key] = input.value;
    });
    return { fields };
}

export function loadFields(state: Record<string, unknown>): boolean {
    const fieldsData = state['fields'];
    if (isRecord(fieldsData)) {
        document.querySelectorAll<HTMLInputElement>('input.persistent-field').forEach((input, index) => {
            const key = input.id || ('f_' + String(index));
            const val = fieldsData[key];
            if (typeof val === 'string') {
                input.value = val;
            }
        });
    }
    syncLinkedFields();
    return false;
}

registerSaveHandler(saveFields);
registerLoadHandler(loadFields);
