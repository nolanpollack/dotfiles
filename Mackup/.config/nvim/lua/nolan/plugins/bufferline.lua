return {
	after = "catppuccin",
	"akinsho/bufferline.nvim",
	dependencies = "nvim-tree/nvim-web-devicons",
	config = function()
		local bufferline = require("bufferline")
		vim.opt.termguicolors = true
		bufferline.setup({
			highlights = require("catppuccin.groups.integrations.bufferline").get(),
			options = {
				diagnostics = "nvim_lsp",
				offsets = {
					{
						filetype = "NvimTree",
						text = "File Explorer",
						highlight = "Directory",
						separator = true, -- use a "true" to enable the default, or set your own character
					},
				},
			},
		})

		vim.keymap.set(
			{
				"n",
				"v",
			},
			"<leader>bb",
			function()
				require("bufferline").pick_buffer()
			end,
			{ desc = "Pick buffer" }
		)
	end,
}
