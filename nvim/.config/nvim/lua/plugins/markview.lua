return {
    -- TODO: Disable by default with keybind to preview in split mode
	"OXY2DEV/markview.nvim",
	dependencies = {
		"echasnovski/mini.icons",
	},
	ft = { "markdown" },
    keys = {
        { "<leader>mv", "<cmd>Markview splitToggle<cr>", desc = "Toggle Markview Preview" },
    },
	opts = {
		preview = {
			filetypes = { "markdown", "Avante" },
            ignore_buftypes = {},
		},
        markdown = {
            enable = false,
        }
	},
}
