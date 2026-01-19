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
		{
			"<leader><leader>",
			function()
				Snacks.picker.smart()
			end,
			desc = "Smart find files",
		},
		{
			"<leader>ff",
			function()
				Snacks.picker()
			end,
			desc = "Find pickers",
		},
		{
			"<leader>fs",
			function()
				Snacks.picker.grep()
			end,
			desc = "Find files",
		},
		{
			"<leader>fh",
			function()
				Snacks.picker.help()
			end,
			desc = "Find help",
		},
		{
			"<leader>fd",
			function()
				Snacks.picker.diagnostics()
			end,
			desc = "Find diagnostics",
		},
		{
			"gr",
			function()
				Snacks.picker.lsp_references()
			end,
			desc = "Go to references",
		},
		{
			"gi",
			function()
				Snacks.picker.lsp_implementations()
			end,
			desc = "Go to implementations",
		},
	},
	lazy = false,
	opts = {
		picker = {
			ui_select = true,
			formatters = {
				file = {
					filename_first = true,
				},
			},
			win = {
				input = {
					keys = {
						["sv"] = { "edit_vsplit", mode = { "n" } },
					},
				},
			},
			sources = {
				files = {
					hidden = true,
				},
			},
		},
	},
}
