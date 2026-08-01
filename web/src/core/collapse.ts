export function updateEmptySections(): void {
    const sections = document.querySelectorAll<HTMLElement>('.d2f-section, .section');
    sections.forEach((sec) => {
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

export function toggleSection(target: HTMLElement | string | null, onSave?: () => void): void {
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
        const isCollapsed = body.classList.toggle('collapsed');
        headerElement.setAttribute('aria-expanded', isCollapsed ? 'false' : 'true');
        const toggler = headerElement.querySelector<HTMLElement>('.stog');
        if (toggler) {
            toggler.innerHTML = isCollapsed ? '&#9650;' : '&#9660;';
        }
        if (onSave) {
            onSave();
        }
    }
}
