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

		local keymap = vim.keymap
		local cinnamon = require("cinnamon")

		keymap.set({ "n", "v" }, "<C-U>", function()
			cinnamon.scroll("<C-U>zz", { mode = "window" })
		end)
		keymap.set({ "n", "v" }, "<C-D>", function()
			cinnamon.scroll("<C-D>zz", { mode = "window" })
		end)

		keymap.set({ "n", "v" }, "{", function()
			cinnamon.scroll("{zz")
		end)
		keymap.set({ "n", "v" }, "}", function()
			cinnamon.scroll("}zz")
		end)
	end,
}
