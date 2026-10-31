// Export targeted references from the user's analyzed GZF working copy.
// Generated listings/decompilation remain under ignored artifacts/, not Git.
// @category Shadowrealm
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.InstructionIterator;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.HexFormat;

public class ExportCompanion extends GhidraScript {
    @Override public void run() throws Exception {
        String[] args=getScriptArgs();
        if(args.length<1)throw new IllegalArgumentException("NEW_OUTPUT_DIRECTORY [hex function entries...]");
        Path output=Path.of(args[0]);Files.createDirectory(output);
        if(!currentProgram.getLanguageID().toString().equals("68000:BE:32:default"))throw new IllegalStateException("Unexpected processor language");
        byte[] bytes=new byte[0x200000];
        if(currentProgram.getMemory().getBytes(toAddr(0),bytes)!=bytes.length)throw new IllegalStateException("ROM memory is incomplete");
        String hash=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
        if(!hash.equals("92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e"))throw new IllegalStateException("Program bytes differ from supplied ROM: "+hash);
        String info="Program: "+currentProgram.getName()+"\nLanguage: "+currentProgram.getLanguageID()+"\nROM SHA256: "+hash+"\nExisting functions: "+currentProgram.getFunctionManager().getFunctionCount()+"\n";
        Files.writeString(output.resolve("program.txt"),info,StandardCharsets.UTF_8);println(info);
        String[] entries=args.length>1?java.util.Arrays.copyOfRange(args,1,args.length):new String[]{"ac84","138b8","14508","145a2","14682","149c0","14ba6","14c2e","15d3c","15dcc","b574","b6e4","bade"};
        DecompInterface decompiler=new DecompInterface();decompiler.openProgram(currentProgram);
        try {
            for(String entry:entries){
                Address address=toAddr(Long.parseUnsignedLong(entry,16));
                Function function=currentProgram.getFunctionManager().getFunctionAt(address);
                boolean added=false;
                if(function==null){disassemble(address);function=createFunction(address,null);added=true;}
                if(function==null){println("No function at "+address);continue;}
                DecompileResults result=decompiler.decompileFunction(function,30,monitor);
                String name="function-"+address;
                String source="/* "+function.getName()+" at "+address+"; created in working copy: "+added+" */\n";
                source+=result.decompileCompleted()?result.getDecompiledFunction().getC():"/* "+result.getErrorMessage()+" */";
                Files.writeString(output.resolve(name+".c"),source,StandardCharsets.UTF_8);
                StringBuilder listing=new StringBuilder();
                InstructionIterator instructions=currentProgram.getListing().getInstructions(function.getBody(),true);
                while(instructions.hasNext()){var instruction=instructions.next();listing.append(instruction.getAddress()).append(" ").append(instruction).append("\n");}
                Files.writeString(output.resolve(name+".asm"),listing,StandardCharsets.UTF_8);
                println("Exported "+function.getName()+" at "+address);
            }
        } finally {decompiler.dispose();}
    }
}
