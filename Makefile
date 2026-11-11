CC     ?= cc
CFLAGS = -Wall -Wextra -std=c11 -g
TARGET = main
SRCS   = main.c circuit.c analysis.c

all: $(TARGET)

$(TARGET): $(SRCS) circuit.h analysis.h
	$(CC) $(CFLAGS) -o $(TARGET) $(SRCS) -lm

clean:
	rm -f $(TARGET)
