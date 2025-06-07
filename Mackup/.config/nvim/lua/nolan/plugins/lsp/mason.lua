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
				"prettier", -- prettier formatter
				"eslint_d", -- eslint linter
				"stylua", -- lua formatter
                "lua_ls", -- lua language server
				"shfmt", -- bash formatter
				"shellcheck", -- bash linter
				"jdtls",
				"java-debug-adapter",
				"typescript-language-server",
			},
		})
	end,
}
