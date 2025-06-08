return {
	after = "catppuccin",
	"akinsho/bufferline.nvim",
	dependencies = "nvim-tree/nvim-web-devicons",
	lazy = false,
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
