//
//  MyTextField.m
//  XXKey
//
//  mist @2025
//

#import "MyTextField.h"
#include <Carbon/Carbon.h>

@implementation MyTextField {
    id eventMonitor;
}

- (void)drawRect:(NSRect)dirtyRect {
    [super drawRect:dirtyRect];
    
    // Drawing code here.
}

- (BOOL)becomeFirstResponder {
    BOOL okToChange = [super becomeFirstResponder];
    if (okToChange) {
        [self setKeyboardFocusRingNeedsDisplayInRect: [self bounds]];
        
        if (!eventMonitor) {
            eventMonitor = [NSEvent addLocalMonitorForEventsMatchingMask:NSKeyDownMask handler:^(NSEvent *event) {
                self.LastKeyCode = event.keyCode;
                //Take the first UTF-16 unit of the typed character (not the first
                //UTF-8 byte), so multi-byte characters survive the round-trip.
                self.LastKeyChar = event.characters.length > 0 ? [event.characters characterAtIndex:0] : 0;
                return event;
            } ];

        }
    }
    return okToChange;
}

-(void) textDidEndEditing:(NSNotification *)notification {
    if (eventMonitor) {
        [NSEvent removeMonitor:eventMonitor];
        eventMonitor = nil;
    }
}

- (void)dealloc {
    //Remove the monitor even if the field was never asked to end editing,
    //otherwise the event monitor (and this object) leaks.
    if (eventMonitor) {
        [NSEvent removeMonitor:eventMonitor];
        eventMonitor = nil;
    }
}

- (void)textDidChange:(NSNotification *)notification {
    if (self.LastKeyCode == kVK_Space) {
        [self setStringValue:@"Space"];
        [self.Parent onMyTextFieldKeyChange:kVK_Space character:kVK_Space];
    } else if (self.LastKeyCode == kVK_Delete || self.LastKeyCode == kVK_ForwardDelete) {
        [self setStringValue:@""];
        [self.Parent onMyTextFieldKeyChange:0xFE character:0xFE];
    } else {
        [self setStringValue:@""];
        [self.Parent onMyTextFieldKeyChange:self.LastKeyCode character:self.LastKeyChar];
        [self setStringValue:[NSString stringWithFormat:@"%c", self.LastKeyChar]];
    }
}

-(void)setTextByChar:(unsigned short)chr {
    if (chr == kVK_Space) {
        [self setStringValue:@"Space"];
    } else if (chr == 0xFE) {
        [self setStringValue:@""];
    } else {
        NSString* str = [NSString stringWithFormat:@"%c", chr];
        [self setStringValue:str];
    }
}
@end
