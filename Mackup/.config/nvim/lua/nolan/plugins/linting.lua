return {
	"mfussenegger/nvim-lint",
	event = { "BufReadPre", "BufNewFile" },
	config = function()
		local lint = require("lint")

		lint.linters_by_ft = {
            bash = { "shellcheck" },
			java = { "checkstyle" },
			javascript = { "eslint_d" },
			javascriptreact = { "eslint_d" },
			python = { "pylint" },
			typescript = { "eslint_d" },
			typescriptreact = { "eslint_d" },
			-- groovy = { "npm-groovy-lint" },
		}

		lint.linters.pylint.cmd = "python"
		lint.linters.pylint.args = { "-m", "pylint", "-f", "json" }
		lint.linters.checkstyle.args = { "-c", "/home/nolan/dotfiles/northeastern_checkstyle.xml" }

		local lint_augroup = vim.api.nvim_create_augroup("lint", { clear = true })

		vim.api.nvim_create_autocmd({ "BufEnter", "BufWritePost", "InsertLeave" }, {
			group = lint_augroup,
			callback = function()
				lint.try_lint()
			end,
		})
	end,
}
