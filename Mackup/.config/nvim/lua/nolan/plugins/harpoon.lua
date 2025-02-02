return {
	"ThePrimeagen/harpoon",
	branch = "harpoon2",
	dependencies = { "nvim-lua/plenary.nvim" },
	keys = {
		{
			"<leader>ha",
			function()
				require("harpoon"):list():add()
			end,
		},
		{
			"<leader>hl",
			function()
				local harpoon = require("harpoon")
				harpoon.ui:toggle_quick_menu(harpoon:list())
			end,
		},
		{
			"<m-h>",
			function()
				require("harpoon"):list():select(1)
			end,
		},
		{
			"<m-j>",
			function()
				require("harpoon"):list():select(2)
			end,
		},
		{
			"<m-k>",
			function()
				require("harpoon"):list():select(3)
			end,
		},
		{
			"<m-l>",
			function()
				require("harpoon"):list():select(4)
			end,
		},
	},
	opts = {},
}
