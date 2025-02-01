return {
	"stevearc/conform.nvim",
	event = { "BufReadPre", "BufNewFile" },
	keys = {
		{
			"<leader>mp",
			function()
				require("conform").format({
					lsp_fallback = true,
					async = true,
					timeout_ms = 1000,
				})
			end,
			{ "n", "v" },
			desc = "Make Pretty (Format file or range in visual mode)",
		},
	},
	opts = {
		formatters_by_ft = {
			css = { "prettier" },
			html = { "prettier" },
			groovy = { "npm-groovy-lint" },
			java = { "google-java-format" },
			javascript = { "prettier" },
			javascriptreact = { "prettier" },
			json = { "prettier" },
			lua = { "stylua" },
			markdown = { "prettier" },
			python = { "isort", "black" },
			rust = { "rustfmt" },
			sh = { "shellcheck" },
			typescript = { "prettier" },
			typescriptreact = { "prettier" },
			yaml = { "prettier" },
		},
        formatters = {
            ["google-java-format"] = {
                prepend_args = { "--aosp" },
            }
        }
	},
}
