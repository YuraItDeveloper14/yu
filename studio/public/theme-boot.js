// Before the page paints: the theme the reader picked in the Studio or in the book.
try {
  const theme = localStorage.getItem('yu-theme');
  if (theme) document.documentElement.dataset.theme = theme;
} catch {
  // No storage in this window: the page keeps its dark default.
}
