return {
	"williamboman/mason-lspconfig.nvim",
	dependencies = {
		"williamboman/mason.nvim",
		"WhoIsSethDaniel/mason-tool-installer.nvim",
	},
	config = function()
		-- import mason
		require("mason").setup({
			ui = {
				icons = {
					package_installed = "✓",
					package_pending = "➜",
					package_uninstalled = "✗",
				},
			},
		})

		require("mason-lspconfig").setup()

		require("mason-tool-installer").setup({
			ensure_installed = {
				"eslint_d", -- eslint linter
				"java-debug-adapter", -- java debugger
				"jdtls", -- java language server
				"prettier", -- prettier formatter
				"shellcheck", -- bash linter
				"shfmt", -- bash formatter
				"stylua", -- lua formatter
				"typescript-language-server",
                "lua_ls", -- lua language server
                "sqls", -- sql language server
			},
		})
	end,
}
