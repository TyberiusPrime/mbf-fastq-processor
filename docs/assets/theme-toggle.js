(function() {
  const THEME_KEY = 'book-theme';
  const LIGHT_THEME = 'light';
  const DARK_THEME = 'dark';
  
  function getTheme() {
    return localStorage.getItem(THEME_KEY) || LIGHT_THEME;
  }
  
  function setTheme(theme) {
    localStorage.setItem(THEME_KEY, theme);
    document.documentElement.setAttribute('data-theme', theme);
    
    // Update toggle button state
    const toggle = document.getElementById('theme-toggle');
    if (toggle) {
      toggle.checked = theme === DARK_THEME;
      toggle.setAttribute('aria-label', theme === DARK_THEME ? 'Switch to light mode' : 'Switch to dark mode');
    }
  }
  
  function toggleTheme() {
    const currentTheme = getTheme();
    const newTheme = currentTheme === LIGHT_THEME ? DARK_THEME : LIGHT_THEME;
    setTheme(newTheme);
  }
  
  // Initialize theme on page load
  document.addEventListener('DOMContentLoaded', function() {
    const savedTheme = getTheme();
    setTheme(savedTheme);
    
    // Add event listener to toggle button
    const toggle = document.getElementById('theme-toggle');
    if (toggle) {
      toggle.addEventListener('change', toggleTheme);
    }
  });
  
  // Set theme immediately to prevent flash
  setTheme(getTheme());
})();