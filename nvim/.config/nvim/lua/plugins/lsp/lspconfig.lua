return {
	{
		"neovim/nvim-lspconfig",
		config = function()
			vim.api.nvim_create_autocmd("LspAttach", {
				callback = function(args)
					local opts = { buffer = args.buf }
					opts.desc = "See available code actions"
					vim.keymap.set({ "n", "v" }, "<tab>", vim.lsp.buf.code_action, opts)

					opts.desc = "Rename"
					vim.keymap.set({ "n", "v" }, "<leader>rn", vim.lsp.buf.rename, opts)

					vim.keymap.set({ "n", "v" }, "<leader>lr", "<cmd>LspRestart<CR>")
				end,
			})

			vim.lsp.inlay_hint.enable()

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
		end,
	},
	{ "williamboman/mason.nvim", opts = {} },
	{
		"mason-org/mason-lspconfig.nvim",
		dependencies = {
			"neovim/nvim-lspconfig",
			"mason-org/mason.nvim",
		},
		opts = {},
	},
	{
		"WhoIsSethDaniel/mason-tool-installer.nvim",
		opts = function()
			return {
				ensure_installed = {
					"eslint-lsp", -- eslint lsp
					"jdtls", -- java language server
					"prettier", -- prettier formatter
					"shellcheck", -- bash linter
					"shfmt", -- bash formatter
					"stylua", -- lua formatter
					"tsgo", -- typescript language server
					"lua_ls", -- lua language server
				},
			}
		end,
	},
	{
		dir = "~/.config/nvim/lua/hover-diagnostics",
		config = function()
			require("hover-diagnostics").setup()
		end,
	},
}
