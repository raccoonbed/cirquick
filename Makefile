CC     = clang
CFLAGS = -Wall -Wextra -std=c11 -g
TARGET = main
SRCS   = main.c circuit.c analysis.c

all: $(TARGET)

$(TARGET): $(SRCS) circuit.h
	$(CC) $(CFLAGS) -lm -o $(TARGET) $(SRCS)

clean:
	rm -f $(TARGET)
