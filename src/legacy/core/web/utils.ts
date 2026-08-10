export interface Utils {
    debounce<T extends (...args: readonly unknown[]) => void>(
        func: T,
        wait: number
    ): (...args: Parameters<T>) => void;
    isRecord(val: unknown): val is Record<string, unknown>;
}

function debounce<T extends (...args: readonly unknown[]) => void>(
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

function isRecord(val: unknown): val is Record<string, unknown> {
    return typeof val === 'object' && val !== null && !Array.isArray(val);
}

window.d2f = window.d2f || {};
window.d2f.utils = {
    debounce,
    isRecord,
}