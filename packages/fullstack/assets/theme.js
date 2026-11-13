(function () {
    const root = document.documentElement;

    // Apply the saved theme on load
    const saved = localStorage.getItem('theme');
    if (saved) root.dataset.theme = saved;

    // Called from the toggle button
    window.toggleTheme = function () {
        const cur = root.dataset.theme
            ?? (matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark');
        const next = cur === 'dark' ? 'light' : 'dark';
        root.dataset.theme = next;
        localStorage.setItem('theme', next);
    };
})();