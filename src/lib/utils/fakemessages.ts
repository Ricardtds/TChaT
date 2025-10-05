import { faker } from "@faker-js/faker";
    
import type { MessageItem } from "$lib/models/MessageItem";

export const fakedata: MessageItem[] = Array.from({ length: 1000 }).map(() => ({
    sender: {
        name: `${faker.person.firstName()}`,
        color: `${faker.color.rgb()}`,
    },
    text: faker.lorem.sentence(),
    time: `${faker.number.int({ max: 23 }).toString().padStart(2, "0")}:${faker.number.int({ max: 59 }).toString().padStart(2, "0")}`,
}));