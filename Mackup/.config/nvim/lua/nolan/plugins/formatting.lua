return {
	"stevearc/conform.nvim",
	opts = function()
		vim.api.nvim_create_autocmd("FileType", {
			pattern = "java",
			command = "setlocal tabstop=2 shiftwidth=2",
		})

		return {
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
			format_on_save = {
				lsp_fallback = true,
				async = false,
				timeout_ms = 1000,
			},
		}
	end,
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
			{ desc = "Make Pretty (Format file or range in visual mode)" },
		},
	},
}
