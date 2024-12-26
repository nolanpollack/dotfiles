-- set leader key
vim.g.mapleader = " "

local keymap = vim.keymap

keymap.set("n", "<leader>nh", ":nohl<CR>", { desc = "Clear search highlights" })
keymap.set("i", "jk", "<ESC>", { desc = "Exit insert mode with jk" })

-- window management
keymap.set("n", "s", "", { desc = "stop s from replacing letter" })
keymap.set("n", "<leader>sv", "<C-w>v", { desc = "Split window vertically" }) -- split window vertically
keymap.set("n", "<leader>sh", "<C-w>s", { desc = "Split window horizontally" }) -- split window horizontally
keymap.set("n", "<leader>se", "<C-w>=", { desc = "Make splits equal size" }) -- make split windows equal width & height
keymap.set("n", "<leader>sx", "<cmd>close<CR>", { desc = "Close current split" }) -- close current split window
keymap.set("n", "<leader>ss", "<C-w>x", { desc = "Swap windows" }) -- swap windows

keymap.set("v", "J", ":m '>+1<CR>gv=gv", { desc = "Move line down in visual mode" })
keymap.set("v", "K", ":m '<-2<CR>gv=gv", { desc = "Move line up in visual mode" })

-- Better motions
keymap.set({ "n", "v", "o" }, "H", "^", { desc = "First non-whitespace character in line" })
keymap.set({ "n", "v", "o" }, "L", "$", { desc = "Last character in line" })
keymap.set({ "n", "v" }, "M", "%", { desc = "Jump to matching bracket" })
keymap.set({ "n", "v" }, "Y", "y$", { desc = "Yank to end of line" })

-- Move in insert mode, And Command-line mode
keymap.set({ "i", "c" }, "<C-j>", "<Down>", { desc = "Move Down in insert mode" })
keymap.set({ "i", "c" }, "<C-k>", "<Up>", { desc = "Move Up in insert mode" })
keymap.set({ "i", "c" }, "<C-h>", "<Left>", { desc = "Move Left in insert mode" })
keymap.set({ "i", "c" }, "<C-l>", "<Right>", { desc = "Move Right in insert mode" })

keymap.set("n", "<leader>rl", ":set relativenumber!<CR>", { desc = "Toggle relative line numbers", silent = true })

local function ansi_colorize()
	local buf = vim.api.nvim_get_current_buf()

	local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
	vim.api.nvim_chan_send(vim.api.nvim_open_term(buf, {}), table.concat(lines, "\r\n"))
end

vim.api.nvim_create_autocmd("BufReadPost", {
	pattern = "*.ansi",
	callback = ansi_colorize, -- Call your Lua function here
})
