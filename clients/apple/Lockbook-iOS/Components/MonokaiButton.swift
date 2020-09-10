//
//  MonokaiButton.swift
//  ios_client
//
//  Created by Parth Mehrotra on 2/9/20.
//  Copyright © 2020 Lockbook. All rights reserved.
//

import SwiftUI

struct MonokaiButton: View {
    var text: String
    var body: some View {
        Text(text)
            .frame(minWidth: 250)
            .padding(10)
            .background(Color(UIColor.label))
            .foregroundColor(Color(UIColor.systemBackground))
            .padding(.bottom, 25)
            .frame(minWidth: 250)
            .font(.system(size: 15, design: .monospaced))
    }
}

struct MonokaiButton_Previews: PreviewProvider {
    static var previews: some View {
        Group {
            MonokaiButton(text: "test")
            MonokaiButton(text: "test")
                .preferredColorScheme(.dark)
        }
    }
}