return {
	"neovim/nvim-lspconfig",
	event = { "BufReadPre", "BufNewFile" },
	dependencies = {
		"williamboman/mason.nvim",
		"hrsh7th/cmp-nvim-lsp",
		{ "antosha417/nvim-lsp-file-operations", config = true },
	},
	config = function()
		-- import lspconfig plugin
		local lspconfig = require("lspconfig")

		-- import mason_lspconfig plugin
		local mason_lspconfig = require("mason-lspconfig")

		-- import cmp-nvim-lsp plugin
		local cmp_nvim_lsp = require("cmp_nvim_lsp")

		local keymap = vim.keymap -- for conciseness

		vim.lsp.inlay_hint.enable()

		vim.api.nvim_create_autocmd("LspAttach", {
			group = vim.api.nvim_create_augroup("UserLspConfig", {}),
			callback = function(ev)
				-- Buffer local mappings.
				-- See `:help vim.lsp.*` for documentation on any of the below functions
				local opts = { buffer = ev.buf, silent = true }

				vim.lsp.handlers["textDocument/hover"] = function(_, result, ctx, _)
					-- Try to peek folded lines under cursor
					local winid = require("ufo").peekFoldedLinesUnderCursor()
					if winid then
						return
					end

					local diagnostics = vim.diagnostic.get(0, { lnum = vim.api.nvim_win_get_cursor(0)[1] - 1 })

					local hover_text
					if not result or not result.contents then
						hover_text = {}
					else
						hover_text = vim.lsp.util.convert_input_to_markdown_lines(result.contents)
					end

					if #diagnostics > 0 then
						table.sort(diagnostics, function(a, b)
							return a.severity > b.severity
						end)
						if #hover_text > 0 then
							table.insert(hover_text, 1, "---")
						end
						for _, diagnostic in ipairs(diagnostics) do
							local severity = diagnostic.severity
							local name = vim.diagnostic.severity[severity]:lower()
							-- Make first character of name upper
							name = name:sub(1, 1):upper() .. name:sub(2)
							local sign = vim.fn.sign_getdefined("DiagnosticSign" .. name)[1].text
							table.insert(hover_text, 1, string.format(" %s %s", sign, diagnostic.message))
						end
					end

					if #hover_text == 0 then
						vim.notify("No information available")
						return
					end

					local bufnr = vim.lsp.util.open_floating_preview(hover_text, "markdown", {
						border = "rounded",
						focus_id = ctx.method,
					})

					if #diagnostics > 0 then
						for i, diagnostic in ipairs(diagnostics) do
							vim.api.nvim_buf_add_highlight(
								bufnr,
								-1,
								"DiagnosticSign" .. vim.diagnostic.severity[diagnostic.severity]:lower(),
								#diagnostics - i,
								0,
								2
							)
						end
					end
				end

				-- vim.keymap.set("n", "K", hover_with_diagnostics, { noremap = true, silent = true })

				opts.desc = "Show LSP definitions"
				keymap.set("n", "gd", "<cmd>Telescope lsp_definitions<CR>", opts) -- show lsp definitions

				opts.desc = "See available code actions"
				keymap.set({ "n", "v" }, "<tab>", vim.lsp.buf.code_action, opts) -- see available code actions, in visual mode will apply to selection

				opts.desc = "Smart rename"
				keymap.set({ "n", "v" }, "<leader>rn", vim.lsp.buf.rename, opts) -- smart rename
			end,
		})

		-- used to enable autocompletion (assign to every lsp server config)
		local capabilities = cmp_nvim_lsp.default_capabilities()

		-- Change the Diagnostic symbols in the sign column (gutter)
		local symbols = { Error = "󰅙", Info = "󰋼", Hint = "󰌵", Warn = "" }
		for type, icon in pairs(symbols) do
			local hl = "DiagnosticSign" .. type
			vim.fn.sign_define(hl, { text = icon, texthl = hl, numhl = hl })
		end

		local function organize_imports()
			local params = {
				command = "_typescript.organizeImports",
				arguments = { vim.api.nvim_buf_get_name(0) },
				title = "",
			}
			vim.lsp.buf.execute_command(params)
		end

		local mason_registry = require("mason-registry")

		mason_lspconfig.setup_handlers({
			-- default handler for installed servers
			function(server_name)
				lspconfig[server_name].setup({
					capabilities = capabilities,
				})
			end,
			["lua_ls"] = function()
				-- configure lua server (with special settings)
				lspconfig["lua_ls"].setup({
					capabilities = capabilities,
					settings = {
						Lua = {
							-- make the language server recognize "vim" global
							diagnostics = {
								globals = { "vim" },
							},
							completion = {
								callSnippet = "Replace",
							},
							hint = {
								enable = true,
							},
						},
					},
				})
			end,
			["clangd"] = function()
				lspconfig["clangd"].setup({
					capabilities = capabilities,
					cmd = {
						"clangd",
						"--fallback-style=webkit", -- so indent uses 4 spaces if no clang format file
					},
				})
			end,
			["ts_ls"] = function()
				lspconfig["ts_ls"].setup({
					capabilities = capabilities,
					commands = {
						OrganizeImports = {
							organize_imports,
							description = "Organize Imports",
						},
					},
					settings = {
						typescript = {
							inlayHints = {
								includeInlayEnumMemberValueHints = true,
								includeInlayFunctionLikeReturnTypeHints = true,
								includeInlayFunctionParameterTypeHints = true,
								includeInlayParameterNameHints = "all",
								includeInlayParameterNameHintsWhenArgumentMatchesName = true,
								includeInlayPropertyDeclarationTypeHints = true,
								includeInlayVariableTypeHints = true,
							},
						},
					},
				})
			end,
			["jdtls"] = function()
				-- Do nothing because we are using nvim-jdtls plugin
			end,
			["groovyls"] = function()
				lspconfig["groovyls"].setup({
					capabilities = capabilities,
					cmd = {
						"java",
						"-jar",
						mason_registry.get_package("groovy-language-server"):get_install_path()
							.. "/build/libs/groovy-language-server-all.jar",
					},
				})
			end,
		})
	end,
}
