return {
	"windwp/nvim-ts-autotag",
	event = { "BufReadPre", "BufNewFile" },
	config = function()
		-- import nvim-ts-autotag plugin
		local autotag = require("nvim-ts-autotag")

		-- configure autotag
		autotag.setup({})
	end,
}
