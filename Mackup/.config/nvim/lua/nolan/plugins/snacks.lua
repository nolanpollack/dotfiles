return {
	"folke/snacks.nvim",
	priority = 1000,
	init = function()
		vim.api.nvim_create_autocmd("User", {
			pattern = "VeryLazy",
			callback = function()
				-- Setup some globals for debugging (lazy-loaded)
				local snacks = require("snacks")
				_G.dd = function(...)
					snacks.debug.inspect(...)
				end
				_G.bt = function()
					snacks.debug.backtrace()
				end
				vim.print = _G.dd -- Override print to use snacks for `:=` command
			end,
		})
	end,
	keys = {
		{
			"<leader>go",
			function()
				require("snacks").gitbrowse()
			end,
			desc = "Open current file in browser",
		},
	},
	lazy = false,
	opts = {},
}
