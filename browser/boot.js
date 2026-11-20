// Keep startup errors visible even when a dependency fails to load.
import('./main.js').catch(error => {
  console.error(error);
  document.querySelector('#status').textContent = 'Could not start the editor.';
  document.querySelector('#loading').hidden = true;
  document.querySelector('#error').textContent = String(error?.message ?? error);
});
