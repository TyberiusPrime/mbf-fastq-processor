(function () {
  function select(element) {
    const selection = window.getSelection();

    const range = document.createRange();
    range.selectNodeContents(element);

    selection.removeAllRanges();
    selection.addRange(range);
  }

  function copyToClipboard(text, button) {
    if (navigator.clipboard) {
      navigator.clipboard.writeText(text).then(() => {
        button.classList.add('copied');
        button.textContent = 'Copied!';
        setTimeout(() => {
          button.classList.remove('copied');
          button.textContent = 'Copy';
        }, 2000);
      });
    }
  }

  document.querySelectorAll("pre").forEach(pre => {
    const code = pre.querySelector('code');
    if (code) {
      // Wrap the pre element in a container for positioning
      const container = document.createElement('div');
      container.className = 'code-container';
      pre.parentNode.insertBefore(container, pre);
      container.appendChild(pre);

      // Create copy button
      const copyButton = document.createElement('button');
      copyButton.className = 'copy-button';
      copyButton.textContent = 'Copy';
      copyButton.setAttribute('aria-label', 'Copy code to clipboard');
      
      copyButton.addEventListener('click', function(event) {
        event.stopPropagation();
        copyToClipboard(code.textContent, copyButton);
      });

      container.appendChild(copyButton);

      // Keep the original click behavior for the code itself
      code.addEventListener("click", function (event) {
        if (window.getSelection().toString()) {
          return;
        }
        select(pre);
        copyToClipboard(code.textContent, copyButton);
      });
    }
  });
})();
