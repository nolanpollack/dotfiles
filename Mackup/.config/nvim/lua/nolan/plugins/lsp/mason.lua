return {
	"williamboman/mason.nvim",
	dependencies = {
		"williamboman/mason-lspconfig.nvim",
		"WhoIsSethDaniel/mason-tool-installer.nvim",
	},
	config = function()
		-- import mason
		local mason = require("mason")

		-- import mason-lspconfig
		local mason_lspconfig = require("mason-lspconfig")

		local mason_tool_installer = require("mason-tool-installer")

		-- enable mason and configure icons
		mason.setup({
			ui = {
				icons = {
					package_installed = "✓",
					package_pending = "➜",
					package_uninstalled = "✗",
				},
			},
		})

		mason_lspconfig.setup({
			-- list of servers for mason to install
			ensure_installed = {
				"bashls", -- Bash
				"clangd", -- C
				"cssls", -- CSS
				"docker_compose_language_service", -- Docker Compose
				"dockerls", -- Docker
				"gradle_ls", -- Gradle
				"html", -- HTML
				"jdtls", -- Java
				"kotlin_language_server", -- Kotlin
				"lua_ls", -- Lua
				"pyright", -- Python
				"tailwindcss", -- Tailwind CSS
				"tsserver", -- JavaScript/TypeScript
			},
		})

		mason_tool_installer.setup({
			ensure_installed = {
				"prettier", -- prettier formatter
				"stylua", -- lua formatter
				"isort", -- python formatter
				"black", -- python formatter
				"shfmt", -- bash formatter
				"shellcheck", -- bash linter
				"pylint",
				"eslint_d",
			},
		})
	end,
}
