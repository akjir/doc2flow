const SECTION_SELECTOR = '.d2f-section, .section';

export interface Collapse {
    updateEmptySections(): void;
    toggleSection(target: HTMLElement | string | null, onSave?: () => void): void;
    saveSections(): Record<string, unknown>;
    loadSections(state: Record<string, unknown>): boolean;
    resetSections(): void;
}

function isRecord(val: unknown): val is Record<string, unknown> {
    return typeof val === 'object' && val !== null && !Array.isArray(val);
}

function setSectionCollapseState(sec: HTMLElement, isCollapsed: boolean): void {
    const body = sec.querySelector<HTMLElement>('.sb');
    const sh = sec.querySelector<HTMLElement>('.sh');
    if (!body) return;
    body.classList.toggle('collapsed', isCollapsed);
    if (sh) {
        sh.setAttribute('aria-expanded', isCollapsed ? 'false' : 'true');
        const toggler = sh.querySelector<HTMLElement>('.stog');
        if (toggler) {
            toggler.innerHTML = isCollapsed ? '&#9650;' : '&#9660;';
        }
    }
}

function updateEmptySections(): void {
    document.querySelectorAll<HTMLElement>(SECTION_SELECTOR).forEach((sec) => {
        const sh = sec.querySelector<HTMLElement>('.sh');
        const body = sec.querySelector<HTMLElement>('.sb');
        if (sh && body && body.children.length === 0 && body.innerHTML.trim() === '') {
            sh.classList.add('no-toggle');
            sh.removeAttribute('role');
            sh.removeAttribute('tabindex');
            sh.removeAttribute('aria-expanded');
        }
    });
}

function toggleSection(target: HTMLElement | string | null, onSave?: () => void): void {
    let headerElement: HTMLElement | null = null;

    if (typeof target === 'string') {
        const sec = document.getElementById(target);
        if (sec) {
            headerElement = sec.querySelector<HTMLElement>('.sh');
        }
    } else if (target instanceof HTMLElement) {
        headerElement = target.closest<HTMLElement>('.sh');
    }

    if (!headerElement || headerElement.classList.contains('no-toggle')) {
        return;
    }

    const section = headerElement.closest<HTMLElement>('.section');
    const body = section ? section.querySelector<HTMLElement>('.sb') : null;

    if (body && (body.children.length > 0 || body.innerHTML.trim() !== '')) {
        const isCollapsed = !body.classList.contains('collapsed');
        if (section) {
            setSectionCollapseState(section, isCollapsed);
        } else {
            body.classList.toggle('collapsed', isCollapsed);
            headerElement.setAttribute('aria-expanded', isCollapsed ? 'false' : 'true');
            const toggler = headerElement.querySelector<HTMLElement>('.stog');
            if (toggler) {
                toggler.innerHTML = isCollapsed ? '&#9650;' : '&#9660;';
            }
        }
        if (onSave) {
            onSave();
        }
    }
}

function saveSections(): Record<string, unknown> {
    const sections: Record<string, boolean> = {};
    document.querySelectorAll<HTMLElement>(SECTION_SELECTOR).forEach((sec, index) => {
        const body = sec.querySelector<HTMLElement>('.sb');
        if (body) {
            const key = sec.id || ('sec_' + String(index));
            sections[key] = body.classList.contains('collapsed');
        }
    });
    return { sections };
}

function loadSections(state: Record<string, unknown>): boolean {
    const sectionsData = state['sections'];
    if (isRecord(sectionsData)) {
        document.querySelectorAll<HTMLElement>(SECTION_SELECTOR).forEach((sec, index) => {
            const key = sec.id || ('sec_' + String(index));
            const shouldCollapse = sectionsData[key];
            if (typeof shouldCollapse === 'boolean') {
                setSectionCollapseState(sec, shouldCollapse);
            }
        });
    }
    return false;
}

function resetSections(): void {
    document.querySelectorAll<HTMLElement>(SECTION_SELECTOR).forEach((sec) => {
        setSectionCollapseState(sec, false);
    });
    document.querySelectorAll<HTMLElement>('.sb.collapsed').forEach((body) => {
        body.classList.remove('collapsed');
    });
}

window.d2f.collapse = {
    updateEmptySections,
    toggleSection,
    saveSections,
    loadSections,
    resetSections,
};

window.d2f.storage.registerSaveHandler(saveSections);
window.d2f.storage.registerLoadHandler(loadSections);
window.d2f.core.registerResetHandler(resetSections);

