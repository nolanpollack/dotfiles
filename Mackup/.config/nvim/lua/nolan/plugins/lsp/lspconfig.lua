-- TODO: Simplify and update (mason too) for neovim 11
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

		vim.lsp.log.set_format_func(vim.inspect)

        -- TODO: Can I make this a plugin?
		vim.api.nvim_create_autocmd("LspAttach", {
			group = vim.api.nvim_create_augroup("UserLspConfig", {}),
			callback = function(ev)
				local opts = { buffer = ev.buf, silent = true }

				local hover_ns = vim.api.nvim_create_namespace("lsp_hover_highlight")
				local hover_with_diagnostics = function()
					-- Try to peek folded lines under cursor
					local winid = require("ufo").peekFoldedLinesUnderCursor()
					if winid then
						return
					end

					local diagnostics = vim.diagnostic.get(0, { lnum = vim.api.nvim_win_get_cursor(0)[1] - 1 })
					local hover_text = {}

					-- Add diagnostics at the top
					if #diagnostics > 0 then
						table.sort(diagnostics, function(a, b)
							return a.severity < b.severity
						end)
						for _, diagnostic in ipairs(diagnostics) do
							local severity = diagnostic.severity
							local name = vim.diagnostic.severity[severity]:lower()
							name = name:sub(1, 1):upper() .. name:sub(2)
							local sign = vim.diagnostic.config().signs.text[diagnostic.severity] or "?"
							table.insert(hover_text, string.format(" %s %s", sign, diagnostic.message))
						end
						table.insert(hover_text, "---")
					end

					-- Make the LSP request
					local params = vim.lsp.util.make_position_params(0, "utf-8")
					vim.lsp.buf_request_all(0, "textDocument/hover", params, function(results)
						local contents = {}
						local client_name_shown = false

						for client_id, response in pairs(results) do
							if response.err then
								vim.notify(response.err.message, vim.log.levels.WARN)
							elseif response.result then
								local client = vim.lsp.get_client_by_id(client_id)
								-- Show client name if multiple results
								if #vim.tbl_keys(results) > 1 and not client_name_shown then
									table.insert(contents, string.format("# %s", client.name))
									client_name_shown = true
								end

								local result = response.result
								-- Handle plaintext vs markdown content
								if type(result.contents) == "table" and result.contents.kind == "plaintext" then
									local lines = vim.split(result.contents.value or "", "\n", { trimempty = true })
									if #results == 1 then
										vim.list_extend(hover_text, lines)
									else
										table.insert(contents, "```")
										vim.list_extend(contents, lines)
										table.insert(contents, "```")
									end
								else
									vim.list_extend(
										contents,
										vim.lsp.util.convert_input_to_markdown_lines(result.contents)
									)
								end

								-- Handle range highlighting
								if result.range then
									local start = result.range.start
									local end_ = result.range["end"]
									local start_idx =
										vim.lsp.util._get_line_byte_from_position(0, start, client.offset_encoding)
									local end_idx =
										vim.lsp.util._get_line_byte_from_position(0, end_, client.offset_encoding)

									vim.highlight.range(
										0,
										hover_ns,
										"LspReferenceText",
										{ start.line, start_idx },
										{ end_.line, end_idx },
										{ priority = vim.highlight.priorities.user }
									)
								end
							end
						end

						-- Extend hover_text with LSP contents
						vim.list_extend(hover_text, contents)

						if #hover_text == 0 then
							vim.notify("No information available")
							return
						end

						-- Show the floating window
						local bufnr = vim.lsp.util.open_floating_preview(hover_text, "markdown", {
							border = "rounded",
							focus_id = "textDocument/hover",
						})

						-- Add highlights for diagnostics
						if #diagnostics > 0 then
							local offset = 0
							for i, diagnostic in ipairs(diagnostics) do
								vim.api.nvim_buf_add_highlight(
									bufnr,
									-1,
									"DiagnosticSign" .. vim.diagnostic.severity[diagnostic.severity]:lower(),
									i - 1 + offset,
									0,
									2
								)
								offset = offset + #vim.split(diagnostic.message, "\n") - 1
							end
						end
					end)
				end

				vim.keymap.set("n", "K", hover_with_diagnostics, { noremap = true, silent = true })

				opts.desc = "Show LSP definitions"
				keymap.set("n", "gd", "<cmd>Telescope lsp_definitions<CR>", opts) -- show lsp definitions

				opts.desc = "See available code actions"
				keymap.set({ "n", "v" }, "<tab>", vim.lsp.buf.code_action, opts) -- see available code actions, in visual mode will apply to selection

				opts.desc = "Smart rename"
				keymap.set({ "n", "v" }, "<leader>rn", vim.lsp.buf.rename, opts) -- smart rename
			end,
		})

		-- Change the Diagnostic symbols in the sign column (gutter)
		vim.diagnostic.config({
			signs = {
				text = {
					[vim.diagnostic.severity.ERROR] = "󰅙",
					[vim.diagnostic.severity.INFO] = "󰋼",
					[vim.diagnostic.severity.HINT] = "󰌵",
					[vim.diagnostic.severity.WARN] = "",
				},
			},
			virtual_text = true,
		})

		vim.lsp.enable({
			"r_language_server",
		})

		-- local function organize_imports()
		-- 	local params = {
		-- 		command = "_typescript.organizeImports",
		-- 		arguments = { vim.api.nvim_buf_get_name(0) },
		-- 		title = "",
		-- 	}
		-- 	vim.lsp.buf.execute_command(params)
		-- end

		-- mason_lspconfig.setup_handlers({
		-- 	-- default handler for installed servers
		-- 	function(server_name)
		-- 		lspconfig[server_name].setup({
		-- 			capabilities = capabilities,
		-- 		})
		-- 	end,
		-- 	["lua_ls"] = function()
		-- 		-- configure lua server (with special settings)
		-- 		lspconfig["lua_ls"].setup({
		-- 			capabilities = capabilities,
		-- 			settings = {
		-- 				Lua = {
		-- 					-- make the language server recognize "vim" global
		-- 					diagnostics = {
		-- 						globals = { "vim" },
		-- 					},
		-- 					completion = {
		-- 						callSnippet = "Replace",
		-- 					},
		-- 					hint = {
		-- 						enable = true,
		-- 					},
		-- 				},
		-- 			},
		-- 		})
		-- 	end,
		-- 	["clangd"] = function()
		-- 		lspconfig["clangd"].setup({
		-- 			capabilities = capabilities,
		-- 			cmd = {
		-- 				"clangd",
		-- 				"--fallback-style=webkit", -- so indent uses 4 spaces if no clang format file
		-- 			},
		-- 		})
		-- 	end,
		-- 	["ts_ls"] = function()
		-- 		lspconfig["ts_ls"].setup({
		-- 			capabilities = capabilities,
		-- 			commands = {
		-- 				OrganizeImports = {
		-- 					organize_imports,
		-- 					description = "Organize Imports",
		-- 				},
		-- 			},
		-- 			settings = {
		-- 				typescript = {
		-- 					inlayHints = {
		-- 						includeInlayEnumMemberValueHints = true,
		-- 						includeInlayFunctionLikeReturnTypeHints = true,
		-- 						includeInlayFunctionParameterTypeHints = true,
		-- 						includeInlayParameterNameHints = "all",
		-- 						includeInlayParameterNameHintsWhenArgumentMatchesName = true,
		-- 						includeInlayPropertyDeclarationTypeHints = true,
		-- 						includeInlayVariableTypeHints = true,
		-- 					},
		-- 				},
		-- 			},
		-- 		})
		-- 	end,
		-- 	["jdtls"] = function()
		-- 		-- Do nothing because we are using nvim-jdtls plugin
		-- 	end,
		-- 	["groovyls"] = function()
		-- 		lspconfig["groovyls"].setup({
		-- 			capabilities = capabilities,
		-- 			cmd = {
		-- 				"java",
		-- 				"-jar",
		-- 				mason_registry.get_package("groovy-language-server"):get_install_path()
		-- 					.. "/build/libs/groovy-language-server-all.jar",
		-- 			},
		-- 		})
		-- 	end,
		-- })
	end,
}
