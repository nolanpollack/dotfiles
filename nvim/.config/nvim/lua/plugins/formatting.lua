return {
	"stevearc/conform.nvim",
	event = { "BufReadPre", "BufNewFile" },
	keys = {
		{
			"<leader>mp",
			function()
				require("conform").format({
					lsp_format = "fallback",
					async = true,
					timeout_ms = 1000,
				})
			end,
			{ "n", "v" },
			desc = "Make Pretty (Format file or range in visual mode)",
		},
	},
	opts = function()
		-- Use two spaces for indentation with prettier-managed files
		vim.api.nvim_create_autocmd("FileType", {
			pattern = {
				"typescript",
				"typescriptreact",
				"javascript",
				"javascriptreact",
				"json",
				"css",
				"html",
				"markdown",
				"yaml",
			},
			callback = function()
				vim.opt_local.shiftwidth = 2
				vim.opt_local.tabstop = 2
				vim.opt_local.softtabstop = 2
			end,
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
				markdown = { "prettier_no_cwd" },
				python = { "isort", "black" },
				r = { "styler" },
				rust = { "rustfmt" },
				sh = { "shellcheck" },
				typescript = { "prettier" },
				typescriptreact = { "prettier" },
				yaml = { "prettier" },
			},
			formatters = {
				["google-java-format"] = {
					prepend_args = { "--aosp" },
				},
				prettier = {
					require_cwd = true,
				},
				prettier_no_cwd = {
					inherit = "prettier",
					require_cwd = false,
				},
			},
		}
	end,
}
