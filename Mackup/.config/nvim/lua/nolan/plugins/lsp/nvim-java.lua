return {
	"nvim-java/nvim-java",
	setup = function()
		local java = require("java")
		java.setup({
			spring_boot_tools = {
				enable = false,
			},
		})
	end,
}
