return {
	"nvim-telescope/telescope.nvim",

	dependencies = {
		"nvim-lua/plenary.nvim",
		{ "nvim-telescope/telescope-fzf-native.nvim", build = "make" },
		"nvim-tree/nvim-web-devicons",
		"folke/todo-comments.nvim",
		"kkharji/sqlite.lua",
		"danielfalk/smart-open.nvim",
		"olacin/telescope-gitmoji.nvim",
	},
	config = function()
		local telescope = require("telescope")
		local actions = require("telescope.actions")

		telescope.setup({
			defaults = {
				path_display = { "smart" },
				mappings = {
					i = {
						["<C-k>"] = actions.move_selection_previous, -- move to prev result
						["<C-j>"] = actions.move_selection_next, -- move to next result
						["<C-q>"] = actions.send_selected_to_qflist + actions.open_qflist,
					},
					n = {
						["sv"] = actions.select_vertical,
						["sh"] = actions.select_horizontal,
					},
				},
			},
			extensions = {
				smart_open = {
					match_algorithm = "fzf",
				},
			},
		})

		telescope.load_extension("fzf")
		telescope.load_extension("smart_open")
		telescope.load_extension("gitmoji")

		-- set keymaps
		local keymap = vim.keymap -- for conciseness

		keymap.set("n", "<leader>fs", "<cmd>Telescope live_grep<cr>", { desc = "Find String in cwd" })
		keymap.set("n", "<leader>fc", "<cmd>Telescope grep_string<cr>", { desc = "Find string under Cursor in cwd" })
		keymap.set("n", "<leader>ft", "<cmd>TodoTelescope<cr>", { desc = "Find Todos" })
		keymap.set("n", "<leader>fb", "<cmd>Telescope buffers<cr>", { desc = "Find Buffers" })
		keymap.set("n", "<leader>fg", "<cmd>Telescope gitmoji<cr>", { desc = "Find Gitmoji for commits" })
		keymap.set("n", "<leader>fp", "<cmd>Telescope builtin<cr>", { desc = "Find telescope Pickers" })
		keymap.set("n", "<leader><leader>", "<cmd>Telescope smart_open<cr>", { desc = "Smart open files" })
	end,
}
