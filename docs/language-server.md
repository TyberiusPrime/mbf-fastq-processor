# mbf-fastq-processor Language Server

The mbf-fastq-processor language server provides IDE features for editing TOML configuration files.

## Features

### 1. Auto-completion

The language server provides intelligent auto-completion for:

- **Step actions**: All available transformation step types (e.g., `Head`, `FilterMinQuality`, `Report`)
- **Section headers**: `[input]`, `[[step]]`, `[output]`, `[options]`, `[barcodes.NAME]`
- **Configuration keys**:
  - Input keys: `read1`, `read2`, `index1`, `index2`
  - Output keys: `prefix`, `format`, `compression`, `report_html`, `report_json`
  - Step keys: `action`, `segment`, `out_label`, `in_label`
  - Options keys: `block_size`, `allow_overwrite`

### 2. Inline Validation

The language server validates your configuration in real-time and shows errors inline:

- **TOML syntax errors**: Catches malformed TOML
- **Configuration errors**: Uses the same validation logic as the main tool
- **Type checking**: Validates that required fields are present and have correct types

### 3. Hover Documentation

Hover over any step action or configuration key to see:

- Detailed descriptions of step actions
- Parameter documentation
- Usage hints

## Installation

### Building the Language Server

```bash
cargo build --release --bin mbf-fastq-processor-lsp
```

The binary will be located at `target/release/mbf-fastq-processor-lsp`.

### Visual Studio Code Setup

1. Install the "Generic Language Server" extension or create a custom extension

2. Add to your VS Code settings (`.vscode/settings.json` or user settings):

```json
{
  "genericLanguageServer.servers": [
    {
      "name": "mbf-fastq-processor",
      "command": "/path/to/mbf-fastq-processor-lsp",
      "args": [],
      "languageIds": ["toml"],
      "initializationOptions": {}
    }
  ]
}
```

### Neovim Setup (using nvim-lspconfig)

Add to your Neovim config:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define the config if it doesn't exist
if not configs.mbf_fastq_processor_lsp then
  configs.mbf_fastq_processor_lsp = {
    default_config = {
      cmd = {'/path/to/mbf-fastq-processor-lsp'},
      filetypes = {'toml'},
      root_dir = lspconfig.util.root_pattern('.git', 'input.toml'),
      settings = {},
    },
  }
end

-- Enable the LSP for TOML files (you might want to check if it's an mbf config)
lspconfig.mbf_fastq_processor_lsp.setup{
  on_attach = function(client, bufnr)
    -- Your on_attach function here
  end,
}
```

### Helix Setup

Add to your `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "toml"
language-servers = ["mbf-fastq-processor-lsp"]

[language-server.mbf-fastq-processor-lsp]
command = "/path/to/mbf-fastq-processor-lsp"
```

### Emacs Setup (using lsp-mode)

Add to your Emacs config:

```elisp
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '(toml-mode . "toml"))

(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "/path/to/mbf-fastq-processor-lsp")
                  :major-modes '(toml-mode)
                  :server-id 'mbf-fastq-processor-lsp))
```

## Usage

Once configured, the language server will automatically activate when you open a TOML file.

### Tips

1. **File association**: You may want to set up file associations so that `input.toml` or `*.mbf.toml` files are recognized automatically

2. **Trigger completion**: Use your editor's completion trigger (usually `Ctrl+Space`) to see available completions

3. **View diagnostics**: Errors and warnings will appear inline as you type. Use your editor's diagnostic navigation commands to jump between issues

4. **Hover for help**: Hover over any step action name to see its documentation

## Troubleshooting

### Language Server Not Starting

Check the language server logs (usually in your editor's LSP log):

```bash
# VSCode: View > Output > Language Server Client
# Neovim: :LspInfo, :LspLog
```

### Completions Not Appearing

- Ensure your cursor is in an appropriate location (after `action = ` or at the start of a line)
- Try manually triggering completion with your editor's completion command

### Validation Not Working

The validation uses `check_for_validation()` which skips file existence checks. This means:
- It will validate configuration structure
- It will NOT complain about missing input files
- It WILL complain about invalid step configurations

## Architecture

The language server is implemented using:

- **tower-lsp**: Async LSP server framework
- **tokio**: Async runtime
- **schemars**: JSON Schema generation for discovering available steps
- **Existing validation**: Reuses mbf-fastq-processor's config validation logic

### Components

- `backend.rs`: Main LSP server implementation
- `completion.rs`: Auto-completion provider
- `diagnostics.rs`: Validation and error reporting
- `hover.rs`: Documentation on hover

## Future Enhancements

Potential future improvements:

1. **Better line number tracking**: Parse TOML with position information to provide exact error locations
2. **Semantic tokens**: Syntax highlighting for step actions, tags, and segments
3. **Code actions**: Quick fixes for common errors
4. **Snippet support**: Smart templates for common step configurations
5. **Go to definition**: Jump to where tags are defined
6. **Rename refactoring**: Rename tags throughout the configuration
7. **Document symbols**: Outline view of configuration sections
