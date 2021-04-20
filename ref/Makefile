SRC_DIR = src
OBJ_DIR = build/obj
SRC = src/main.cpp src/keyboard.cpp
OBJ = $(patsubst src/%, $(OBJ_DIR)/%, $(patsubst %.cpp, %.o, $(SRC)))
PDCURSES_DIR = third_party/pdcurses
WIN_PDCURSES = $(PDCURSES_DIR)/wincon/pdcurses.a
INCLUDES = -I$(PDCURSES_DIR)

build/prog: $(OBJ) $(WIN_PDCURSES)
	@mkdir -p build
	@echo linking $@
	@g++ $^ -o $@

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.cpp
	@mkdir -p $(OBJ_DIR)
	@echo compiling $(notdir $<)
	@g++ -c $^ $(INCLUDES) --std=c++17  -o $@

$(WIN_PDCURSES):
	@echo build $(notdir $@)
	@cd $(PDCURSES_DIR)/wincon && make
