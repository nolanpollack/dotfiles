return {
	after = "catppuccin",
	"akinsho/bufferline.nvim",
	dependencies = "nvim-tree/nvim-web-devicons",
	config = function()
		local bufferline = require("bufferline")
		vim.opt.termguicolors = true
		bufferline.setup({
			highlights = require("catppuccin.groups.integrations.bufferline").get(),
			options = {
				diagnostics = "nvim_lsp",
				diagnostics_indicator = function(_, _, diagnostics_dict, _)
					local s = " "
					for e, n in pairs(diagnostics_dict) do
						local sym = e == "error" and " "
							or (
								e == "warning" and " "
								or (e == "info" and "󰋼 " or (e == "hint" and "󰌵 " or " "))
							)
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
		})

		vim.keymap.set(
			{
				"n",
				"v",
			},
			"<leader>bb",
			function()
				require("bufferline").pick_buffer()
			end,
			{ desc = "Pick buffer" }
		)
		vim.keymap.set({ "n", "v" }, "<leader>bd", ":bdelete<CR>", { desc = "Delete buffer" })
		vim.keymap.set({ "n", "v" }, "<leader>bse", ":BufferLineSortByExtension<CR>", { desc = "Sort by extension" })
		vim.keymap.set({ "n", "v" }, "<leader>bsn", ":BufferLineSortByDirectory<CR>", { desc = "Sort by directory" })
	end,
}
