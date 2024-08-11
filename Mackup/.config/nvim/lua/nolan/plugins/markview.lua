return {
	"OXY2DEV/markview.nvim",
	lazy = false,
	dependencies = {
		"echasnovski/mini.icons",
	},

	config = function()
		require("markview").setup()
	end,
}
