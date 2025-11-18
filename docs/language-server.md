# mbf-fastq-processor Language Server

The mbf-fastq-processor language server provides IDE features for editing TOML configuration files.

## Features

### 1. Auto-completion with Snippets

The language server provides intelligent auto-completion for:

- **Step actions**: All available transformation step types
  - Basic completions: Just the action name (e.g., type `"Head"` for a simple `action = "Head"`)
  - **Template-based snippets**: Full examples from `template.toml` for common steps
    - `[[step]] - Head`, `[[step]] - Report`, `[[step]] - ExtractRegions`, `[[step]] - ExtractIUPAC`
    - `[[step]] - FilterMinQuality`, `[[step]] - CutStart`, `[[step]] - Truncate`
    - And many more with complete parameter examples
- **Section headers with templates**:
  - `[input]` and `[output]` sections use the canonical `template.toml` examples
  - Each section includes all common parameters with tab stops
  - `[[step]]`, `[options]`, `[barcodes.NAME]` for other sections
- **Configuration keys with snippets**:
  - Input keys: `read1`, `read2`, `index1`, `index2` (with filename placeholders)
  - Output keys: `prefix`, `format`, `compression`, `report_html`, `report_json` (with value suggestions)
  - Step keys: `action`, `segment`, `out_label`, `in_label` (with smart defaults)
  - Options keys: `block_size`, `allow_overwrite`

**Snippet Features**:
- **Template-powered**: Snippets use the same canonical templates as the CLI's `mbf-fastq-processor template` command
- **Tab stops**: Press Tab to jump between placeholders (`${1}`, `${2}`, etc.)
- **Choices**: Some fields offer dropdown choices (e.g., `segment` offers `read1|read2|index1|index2|all`)
- **Default values**: Real examples from template.toml like `${1:fileA_1.fastq}` or `${1:umi}`
- **Multi-line templates**: Section headers and steps insert complete, working examples with proper indentation

### 2. Inline Validation with Precise Error Location

The language server validates your configuration in real-time and shows errors inline:

- **TOML syntax errors**: Catches malformed TOML with accurate line and column information
- **Configuration errors**: Uses the same validation logic as the main tool
- **Smart error positioning**:
  - Errors are highlighted at the relevant section (e.g., `[[step]]`, `[input]`, `[output]`)
  - Step-specific errors point to the exact step number
  - Line and column information extracted from parser errors
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

### Snippet Examples

**Creating a complete input section**:
1. Type `[in` and trigger completion
2. Select `[input]` from the list
3. The template expands to:
   ```toml
   [input]
       read1 = ['input_R1.fastq.gz']
   ```
4. The filename is highlighted - just type to replace it
5. Press Tab to move to the next field (or Esc to finish)

**Adding a quality report step**:
1. Type `[[step` or `action =` and trigger completion
2. Select `[[step]] - Report` from the list
3. The complete template (from `template.toml`) expands with all parameters
4. Tab through each field to customize values

**Adding an ExtractIUPAC step**:
1. Select `[[step]] - ExtractIUPAC` from completions
2. Get a complete example with:
   - `action = "ExtractIUPAC"`
   - `out_label`, `search`, `max_mismatches`, `anchor`, `segment` with example values
3. All values are editable placeholders from the canonical template

**Using choice snippets**:
- When you add `segment = `, you'll get a dropdown with `read1|read2|index1|index2|all`
- When you add `compression = `, you'll get `Uncompressed|Gzip|Zstd`
- Choose with arrow keys and Enter, or just type the value

### Tips

1. **File association**: You may want to set up file associations so that `input.toml` or `*.mbf.toml` files are recognized automatically

2. **Trigger completion**: Use your editor's completion trigger (usually `Ctrl+Space`) to see available completions
   - VSCode/Neovim: `Ctrl+Space`
   - Some editors also trigger on `[`, `"`, or `=`

3. **Navigate snippets**: After inserting a snippet:
   - Press `Tab` to jump to the next placeholder
   - Press `Shift+Tab` to go back
   - Press `Esc` to exit snippet mode

4. **View diagnostics**: Errors and warnings will appear inline as you type. Use your editor's diagnostic navigation commands to jump between issues
   - VSCode: `F8` (next error), `Shift+F8` (previous error)
   - Neovim: `]d` and `[d` (with typical LSP config)

5. **Hover for help**: Hover over any step action name to see its documentation

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
