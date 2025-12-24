#include <windows.h>
#include <stdio.h>

int a = 0;

DWORD WINAPI ThreadFunc(void* data) {
	// Do stuff.  This will be the first function called on the new thread.
	// When this function returns, the thread goes away.  See MSDN for more details.
	for (size_t i = 0; i < 1000; i++) {
		printf("%d", a);
	}
	return 0;
}

int main() {
  HANDLE thread = CreateThread(NULL, 0, ThreadFunc, NULL, 0, NULL);
  if (thread) {
	  for(size_t i = 0; i < 1000; i++) {
		  a++;
	  }
  }
}
