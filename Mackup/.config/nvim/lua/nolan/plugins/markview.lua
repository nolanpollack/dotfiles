return {
    -- TODO: Disable by default with keybind to preview in split mode
	"OXY2DEV/markview.nvim",
	dependencies = {
		"echasnovski/mini.icons",
	},
	ft = { "markdown" },
	opts = {
		preview = {
			filetypes = { "markdown", "Avante" },
            ignore_buftypes = {},
		},
	},
}
