return {
	"declancm/cinnamon.nvim",
	version = "*",
	options = {},
	config = function()
		require("cinnamon").setup({
			keymaps = {
				basic = true,
			},
			options = {
				max_delta = {
					time = 150,
				},
				mode = "window",
			},
		})
	end,
}
