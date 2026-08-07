function formatDateFromTemplate(now: Date, template?: string): string | null {
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

function getTodayFormatted(): string {
    const i18n = window.d2f.lang.dictionary;
    const now = new Date();
    try {
        const fromTemplate = formatDateFromTemplate(now, i18n.date_placeholder);
        if (fromTemplate) return fromTemplate;
    } catch (e) {
        console.warn('Failed to format date', e);
    }
    return now.toLocaleDateString(navigator.language || undefined);
}

function checkDateShortcut(input: HTMLInputElement): boolean {
    if (typeof input.value !== 'string') return false;
    if (input.value.trim().toLowerCase() === 'today') {
        input.value = getTodayFormatted();
        return true;
    }
    return false;
}

export interface Fields {
    formatDateFromTemplate(now: Date, template?: string): string | null;
    getTodayFormatted(): string;
    checkDateShortcut(input: HTMLInputElement): boolean;
    saveFields(): Record<string, unknown>;
    loadFields(state: Record<string, unknown>): boolean;
    resetFields(): void;
}

function saveFields(): Record<string, unknown> {
    const fields: Record<string, string> = {};
    document.querySelectorAll<HTMLInputElement>('input.persistent-field').forEach((input, index) => {
        const key = input.id || ('f_' + String(index));
        fields[key] = input.value;
    });
    return { fields };
}

function loadFields(state: Record<string, unknown>): boolean {
    const fieldsData = state['fields'];
    if (window.d2f.utils.isRecord(fieldsData)) {
        document.querySelectorAll<HTMLInputElement>('input.persistent-field').forEach((input, index) => {
            const key = input.id || ('f_' + String(index));
            const val = fieldsData[key];
            if (typeof val === 'string') {
                input.value = val;
                input.setAttribute('value', val);
            }
        });
    }
    return false;
}

function resetFields(): void {
    document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>(
        'input, textarea, select'
    ).forEach((el) => {
        if (el.id === 'search-input' || el.classList.contains('search-input')) return;
        if (el instanceof HTMLInputElement) {
            if (el.type === 'checkbox' || el.type === 'radio') {
                el.checked = false;
                el.removeAttribute('checked');
            } else {
                el.value = '';
                el.removeAttribute('value');
            }
        } else if (el instanceof HTMLTextAreaElement) {
            el.value = '';
            el.textContent = '';
            el.removeAttribute('value');
        } else if (el instanceof HTMLSelectElement) {
            el.selectedIndex = 0;
        }
    });
}

if (typeof window !== 'undefined') {
    window.d2f.storage.registerSaveHandler(saveFields);
    window.d2f.storage.registerLoadHandler(loadFields);

    document.addEventListener('DOMContentLoaded', () => {
        window.d2f.core.registerResetHandler(resetFields);
    });

    const saveStateDebounced = window.d2f.utils.debounce(() => window.d2f.storage.saveState(), 300);

    const handleInputOrChange = (e: Event): void => {
        const target = e.target;
        if (!(target instanceof HTMLInputElement)) return;

        if (target.classList.contains('persistent-field')) {
            target.setAttribute('value', target.value);
            saveStateDebounced();
        }

        if (target.matches('input[id*="date"], input[name*="date"], input.date-field')) {
            checkDateShortcut(target);
            target.setAttribute('value', target.value);
            saveStateDebounced();
        }
    };

    document.addEventListener('input', handleInputOrChange);
    document.addEventListener('change', handleInputOrChange);
}