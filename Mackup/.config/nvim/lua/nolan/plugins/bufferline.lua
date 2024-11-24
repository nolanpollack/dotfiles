return {
	after = "catppuccin",
	"akinsho/bufferline.nvim",
	dependencies = "nvim-tree/nvim-web-devicons",
	lazy = false,
	keys = {
		{
			"<leader>bb",
			function()
				require("bufferline").pick_buffer()
			end,
			desc = "Pick buffer",
			mode = { "n", "v" },
		},
		{ "<leader>bd", ":bdelete<CR>", desc = "Delete buffer", mode = { "n", "v" } },
		{ "<leader>bse", ":BufferLineSortByExtension<CR>", desc = "Sort by extension", mode = { "n", "v" } },
		{ "<leader>bsn", ":BufferLineSortByDirectory<CR>", desc = "Sort by directory", mode = { "n", "v" } },
	},
	opts = {
		options = {
			always_show_bufferline = false,
			diagnostics = "nvim_lsp",
			diagnostics_indicator = function(_, _, diagnostics_dict, _)
				local s = " "
				for e, n in pairs(diagnostics_dict) do
					local sym = e == "error" and " "
						or (e == "warning" and " " or (e == "info" and "󰋼 " or (e == "hint" and "󰌵 " or " ")))
					s = s .. n .. " " .. sym
				end
				return s
			end,

			offsets = {
				{
					filetype = "NvimTree",
					text = "File Explorer",
					highlight = "Directory",
					separator = true, -- use a "true" to enable the default, or set your own character
				},
			},
		},
	},
}
