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

keymap.set("v", "J", ":m '>+1<CR>gv=gv", { desc = "Move line down in visual mode" })
keymap.set("v", "K", ":m '<-2<CR>gv=gv", { desc = "Move line up in visual mode" })

-- Scrolling
keymap.set({ "n", "v" }, "<C-u>", "<C-u>zz", { desc = "Better half up scroll", remap = true })
keymap.set({ "n", "v" }, "<C-d>", "<C-d>zz", { desc = "Better half down scroll", remap = true })

keymap.set("n", "<leader>a", "gg<S-v>G", { desc = "Select all" })

-- Move in insert mode, And Command-line mode
keymap.set({ "i", "c" }, "<C-j>", "<Down>", { desc = "Move Down in insert mode" })
keymap.set({ "i", "c" }, "<C-k>", "<Up>", { desc = "Move Up in insert mode" })
keymap.set({ "i", "c" }, "<C-h>", "<Left>", { desc = "Move Left in insert mode" })
keymap.set({ "i", "c" }, "<C-l>", "<Right>", { desc = "Move Right in insert mode" })
